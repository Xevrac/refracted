using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_GarrisonPlayEffectAtWindows
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.GarrisonPlayEffectAtWindows); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.GarrisonPlayEffectAtWindows)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize GarrisonId
            s.Write(value.GarrisonId);
            //  Serialize EffectName
            s.Write(value.EffectName);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.GarrisonPlayEffectAtWindows)) as Rts.CnC.Messages.Client.GarrisonPlayEffectAtWindows;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize GarrisonId
            s.Read(out value.GarrisonId);
            //  Deserialize EffectName
            s.Read(out value.EffectName);

            return value;
        }
        
    }
}
