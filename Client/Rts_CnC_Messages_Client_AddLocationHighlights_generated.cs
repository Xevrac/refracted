using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_AddLocationHighlights
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.AddLocationHighlights); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.AddLocationHighlights)obj;
            //  Serialize array Locations
            Rts.Serialization.Reference.Write(s, value.Locations, () =>
            {
                s.WriteVarInt32(value.Locations.Length);
                for(int i = 0 ; i < value.Locations.Length ; ++i)
                {
                    Rts.Serialization.Reference.Write(s, value.Locations[i], () =>
                    {
                        GeneratedSerializers.CurrentVersion.System_Tuple_SlimMath_Vector3_System_Single.Serializer.Serialize(s, value.Locations[i]);
                    });
                }
            });

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.AddLocationHighlights)) as Rts.CnC.Messages.Client.AddLocationHighlights;
            //  Deserialize array Locations
            Rts.Serialization.Reference.Read(s, out value.Locations, () =>
            {
                int length = s.ReadVarInt32();
                System.Tuple<SlimMath.Vector3,System.Single>[] tmp = new System.Tuple<SlimMath.Vector3,System.Single>[length];
                for(int i = 0 ; i < length ; ++i)
                {
                    Rts.Serialization.Reference.Read(s, out tmp[i], () =>
                    {
                        return GeneratedSerializers.CurrentVersion.System_Tuple_SlimMath_Vector3_System_Single.Serializer.Deserialize(s) as System.Tuple<SlimMath.Vector3,System.Single>;
                    });
                }
                return tmp;
            });

            return value;
        }
        
    }
}
