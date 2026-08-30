using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestUngarrisonAll
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestUngarrisonAll); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestUngarrisonAll)obj;
            //  Serialize GarrisonPlayerId
            s.Write(value.GarrisonPlayerId);
            //  Serialize GarrisonId
            s.Write(value.GarrisonId);
            //  Serialize ModifierFlags
            s.WriteEnum(value.ModifierFlags);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestUngarrisonAll)) as Rts.CnC.Messages.Client.RequestUngarrisonAll;
            //  Deserialize GarrisonPlayerId
            s.Read(out value.GarrisonPlayerId);
            //  Deserialize GarrisonId
            s.Read(out value.GarrisonId);
            //  Deserialize ModifierFlags
            s.ReadEnum(out value.ModifierFlags);

            return value;
        }
        
    }
}
