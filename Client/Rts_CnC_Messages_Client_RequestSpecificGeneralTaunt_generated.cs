using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestSpecificGeneralTaunt
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestSpecificGeneralTaunt); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestSpecificGeneralTaunt)obj;
            //  Serialize TauntName
            s.Write(value.TauntName);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestSpecificGeneralTaunt)) as Rts.CnC.Messages.Client.RequestSpecificGeneralTaunt;
            //  Deserialize TauntName
            s.Read(out value.TauntName);

            return value;
        }
        
    }
}
