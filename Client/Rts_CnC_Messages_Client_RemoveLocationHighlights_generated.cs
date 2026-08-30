using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RemoveLocationHighlights
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RemoveLocationHighlights); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RemoveLocationHighlights)obj;
            //  Serialize array Locations
            Rts.Serialization.Reference.Write(s, value.Locations, () =>
            {
                s.WriteVarInt32(value.Locations.Length);
                for(int i = 0 ; i < value.Locations.Length ; ++i)
                {
                    s.Write(value.Locations[i]);
                }
            });

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RemoveLocationHighlights)) as Rts.CnC.Messages.Client.RemoveLocationHighlights;
            //  Deserialize array Locations
            Rts.Serialization.Reference.Read(s, out value.Locations, () =>
            {
                int length = s.ReadVarInt32();
                SlimMath.Vector3[] tmp = new SlimMath.Vector3[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    s.Read(out tmp[i]);
                }
                return tmp;
            });

            return value;
        }
        
    }
}
